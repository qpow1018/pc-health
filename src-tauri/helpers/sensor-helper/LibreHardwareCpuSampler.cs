using LibreHardwareMonitor.Hardware;

namespace PcHealth.SensorHelper;

public sealed class LibreHardwareCpuSampler : ICpuSensorSampler, IDisposable
{
    private readonly Computer computer = new()
    {
        IsCpuEnabled = true,
    };

    public LibreHardwareCpuSampler()
    {
        computer.Open();
    }

    public IReadOnlyList<HardwareRecord> Sample(ICollection<string> errors)
    {
        var hardwareRecords = new List<HardwareRecord>();
        foreach (var hardware in computer.Hardware)
        {
            CollectHardware(hardware, null, hardwareRecords, errors);
        }

        return hardwareRecords;
    }

    public void Dispose()
    {
        computer.Close();
    }

    private static void CollectHardware(
        IHardware hardware,
        string? parentIdentifier,
        ICollection<HardwareRecord> hardwareRecords,
        ICollection<string> errors)
    {
        string? updateError = null;
        try
        {
            hardware.Update();
        }
        catch (Exception exception)
        {
            updateError = exception.Message;
            errors.Add($"Hardware {hardware.Identifier} update failed: {exception.Message}");
        }

        var identifier = hardware.Identifier.ToString();
        var sensors = hardware.Sensors
            .Select(sensor => new SensorRecord(
                sensor.Name,
                sensor.SensorType.ToString(),
                sensor.Identifier.ToString(),
                identifier,
                SensorUnits.For(sensor.SensorType),
                SensorValues.Normalize(sensor.Value, errors, sensor.Identifier.ToString())))
            .ToList();

        hardwareRecords.Add(new HardwareRecord(
            hardware.Name,
            hardware.HardwareType.ToString(),
            identifier,
            parentIdentifier,
            updateError,
            sensors));

        foreach (var subHardware in hardware.SubHardware)
        {
            CollectHardware(subHardware, identifier, hardwareRecords, errors);
        }
    }
}
